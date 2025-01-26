export const getBar = async () => {
  const { foo } = await import('@/foo');
  return foo + 1;
}